//! Spec: display-agent. Runs the real agent script with `sh` against a local
//! display stream, with short timings and a fake `swaymsg` that records its
//! arguments (design decision 7).

#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::convert::Infallible;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use axum::Router;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use futures::{Stream, StreamExt, stream};
use tokio::sync::watch;
use tokio::task::JoinHandle;

const ON: &str = "output * power on";
const OFF: &str = "output * power off";

/// How the test backend behaves on a connection.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// The decision, its changes and heartbeats.
    Normal,
    /// The decision, then the connection ends.
    CloseAfterFirst,
    /// The decision, then nothing at all, not even heartbeats.
    Silent,
}

#[derive(Clone)]
struct Backend {
    decision: watch::Sender<&'static str>,
    mode: Mode,
    connections: Arc<AtomicUsize>,
    /// Ends every open stream when set.
    kill: watch::Sender<bool>,
}

/// A display stream on a fixed port that can be stopped and started again.
struct TestBackend {
    port: u16,
    backend: Backend,
    server: Option<JoinHandle<()>>,
}

impl TestBackend {
    async fn start(decision: &'static str, mode: Mode) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut test = Self {
            port,
            backend: Backend {
                decision: watch::Sender::new(decision),
                mode,
                connections: Arc::new(AtomicUsize::new(0)),
                kill: watch::Sender::new(false),
            },
            server: None,
        };
        test.serve(listener);
        test
    }

    /// A port where nothing answers.
    async fn unreachable() -> Self {
        let mut test = Self::start("on", Mode::Normal).await;
        test.stop();
        test
    }

    fn serve(&mut self, listener: tokio::net::TcpListener) {
        self.backend.kill.send_replace(false);
        let app = Router::new()
            .route("/api/v1/display", get(display))
            .with_state(self.backend.clone());
        self.server = Some(tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        }));
    }

    /// Stops listening and ends every open stream.
    fn stop(&mut self) {
        self.backend.kill.send_replace(true);
        if let Some(server) = self.server.take() {
            server.abort();
        }
    }

    async fn restart(&mut self, decision: &'static str) {
        self.backend.decision.send_replace(decision);
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", self.port))
            .await
            .unwrap();
        self.serve(listener);
    }

    fn set(&self, decision: &'static str) {
        self.backend.decision.send_replace(decision);
    }

    fn connections(&self) -> usize {
        self.backend.connections.load(Ordering::SeqCst)
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

async fn display(
    State(backend): State<Backend>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    backend.connections.fetch_add(1, Ordering::SeqCst);
    let event = |data: &str| Ok(Event::default().event("display").data(data));
    let first = stream::once(futures::future::ready(event(*backend.decision.borrow())));
    let rest = match backend.mode {
        Mode::Normal => changes(backend.decision.subscribe()).map(event).boxed(),
        Mode::CloseAfterFirst => stream::empty().boxed(),
        Mode::Silent => stream::pending().boxed(),
    };
    let mut kill = backend.kill.subscribe();
    let ended = async move {
        let _ = kill.wait_for(|kill| *kill).await;
    };
    let events = first.chain(rest).take_until(ended);
    let keep_alive = match backend.mode {
        Mode::Silent => KeepAlive::new().interval(Duration::from_secs(3600)),
        _ => KeepAlive::new().interval(Duration::from_millis(200)),
    };
    Sse::new(events).keep_alive(keep_alive)
}

/// The values a watch channel takes after now.
fn changes(receiver: watch::Receiver<&'static str>) -> impl Stream<Item = &'static str> {
    stream::unfold(receiver, |mut receiver| async move {
        receiver.changed().await.ok()?;
        let value = *receiver.borrow_and_update();
        Some((value, receiver))
    })
}

/// How the fake `swaymsg` behaves.
#[derive(Clone, Copy)]
enum Swaymsg {
    Records,
    /// Every call fails, like a stale sway socket.
    Fails,
    /// The first call fails, later ones succeed.
    FailsOnce,
}

/// The agent script, run with a fake `swaymsg` first in `PATH`.
struct Agent {
    child: Child,
    calls: PathBuf,
    /// The `sleep` calls, when they are counted.
    sleeps: PathBuf,
    dir: PathBuf,
}

impl Agent {
    fn start(url: &str, failsafe_seconds: u32) -> Self {
        Self::start_with(url, failsafe_seconds, Swaymsg::Records)
    }

    fn start_with(url: &str, failsafe_seconds: u32, fake: Swaymsg) -> Self {
        Self::spawn(url, failsafe_seconds, 2, fake, false)
    }

    /// An agent with the given silence limit whose `sleep` calls are
    /// counted. Only where needed: on macOS every new executable's first
    /// run is slow, which would delay the other tests.
    fn start_counting_sleeps(url: &str, silence_seconds: u32) -> Self {
        Self::spawn(url, 600, silence_seconds, Swaymsg::Records, true)
    }

    fn spawn(
        url: &str,
        failsafe_seconds: u32,
        silence_seconds: u32,
        fake: Swaymsg,
        count_sleeps: bool,
    ) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "jukelight-agent-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let calls = dir.join("calls");
        std::fs::write(&calls, "").unwrap();
        let swaymsg = dir.join("swaymsg");
        // Every call is recorded; a failing fake then exits with status 1.
        let record = format!("printf '%s\\n' \"$*\" >> '{}'", calls.display());
        let body = match fake {
            Swaymsg::Records => record,
            Swaymsg::Fails => format!("{record}\nexit 1"),
            Swaymsg::FailsOnce => format!(
                "{record}\n[ \"$(wc -l < '{calls}')\" -gt 1 ] || exit 1",
                calls = calls.display()
            ),
        };
        std::fs::write(&swaymsg, format!("#!/bin/sh\n{body}\n")).unwrap();
        Command::new("chmod")
            .arg("+x")
            .arg(&swaymsg)
            .status()
            .unwrap();

        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../host/display-agent/jukelight-display-agent");
        let system_path = std::env::var("PATH").unwrap_or_default();
        let sleeps = dir.join("sleeps");
        if count_sleeps {
            // Records the call, then runs the real `sleep`.
            std::fs::write(&sleeps, "").unwrap();
            let sleep = dir.join("sleep");
            std::fs::write(
                &sleep,
                format!(
                    "#!/bin/sh\necho \"$*\" >> '{}'\nPATH='{system_path}' exec sleep \"$@\"\n",
                    sleeps.display()
                ),
            )
            .unwrap();
            Command::new("chmod")
                .arg("+x")
                .arg(&sleep)
                .status()
                .unwrap();
        }
        let path = format!("{}:{system_path}", dir.display());
        let child = Command::new("sh")
            .arg(script)
            .env("PATH", path)
            .env("JUKELIGHT_URL", url)
            .env("RETRY_SECONDS", "1")
            .env("SILENCE_SECONDS", silence_seconds.to_string())
            .env("FAILSAFE_SECONDS", failsafe_seconds.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            // Its own process group, so stopping it also stops its curl.
            .process_group(0)
            .spawn()
            .unwrap();
        Self {
            child,
            calls,
            sleeps,
            dir,
        }
    }

    /// The agent's exit status, if it ends within `limit`.
    async fn exit_status(&mut self, limit: Duration) -> Option<std::process::ExitStatus> {
        let deadline = Instant::now() + limit;
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait().unwrap() {
                return Some(status);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        None
    }

    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(&self.calls)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    /// How many times the agent has started `sleep`.
    fn sleeps(&self) -> usize {
        std::fs::read_to_string(&self.sleeps)
            .unwrap()
            .lines()
            .count()
    }

    /// Waits until the recorded calls match, for at most `limit`.
    async fn wait_for(&self, expected: &[&str], limit: Duration) {
        let deadline = Instant::now() + limit;
        while self.calls() != expected {
            assert!(
                Instant::now() < deadline,
                "expected {expected:?}, recorded {:?}",
                self.calls()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

impl Drop for Agent {
    /// Kills the agent's whole process group (curl, watchdog, sleep), with
    /// SIGKILL so no deferred trap can keep it alive, and never waits long:
    /// a stuck clean-up must not hang the test run.
    fn drop(&mut self) {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", self.child.id())])
            .stderr(Stdio::null())
            .status();
        let deadline = Instant::now() + Duration::from_secs(5);
        while matches!(self.child.try_wait(), Ok(None)) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

const SOON: Duration = Duration::from_secs(3);

mod follow_the_display_decisions {
    use super::*;

    #[tokio::test]
    async fn decision_on() {
        let backend = TestBackend::start("on", Mode::Normal).await;
        let agent = Agent::start(&backend.url(), 600);
        agent.wait_for(&[ON], SOON).await;
    }

    #[tokio::test]
    async fn decision_off() {
        let backend = TestBackend::start("off", Mode::Normal).await;
        let agent = Agent::start(&backend.url(), 600);
        agent.wait_for(&[OFF], SOON).await;
    }

    #[tokio::test]
    async fn changes_follow_and_repeats_are_ignored() {
        let backend = TestBackend::start("on", Mode::Normal).await;
        let agent = Agent::start(&backend.url(), 600);
        agent.wait_for(&[ON], SOON).await;
        backend.set("off");
        agent.wait_for(&[ON, OFF], SOON).await;
        backend.set("on");
        agent.wait_for(&[ON, OFF, ON], SOON).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert_eq!(agent.calls(), [ON, OFF, ON], "heartbeats change nothing");
    }
}

mod apply_the_decision_on_connect {
    use super::*;

    #[tokio::test]
    async fn reconnect_with_the_same_decision() {
        let backend = TestBackend::start("off", Mode::CloseAfterFirst).await;
        let agent = Agent::start(&backend.url(), 600);
        agent.wait_for(&[OFF, OFF], SOON).await;
        assert!(backend.connections() >= 2);
    }
}

mod reconnect_to_the_backend {
    use super::*;

    #[tokio::test]
    async fn backend_restarts() {
        let mut backend = TestBackend::start("off", Mode::Normal).await;
        let agent = Agent::start(&backend.url(), 600);
        agent.wait_for(&[OFF], SOON).await;

        backend.stop();
        tokio::time::sleep(Duration::from_secs(3)).await;
        backend.restart("on").await;
        let restarted = Instant::now();
        agent.wait_for(&[OFF, ON], Duration::from_secs(2)).await;
        assert!(restarted.elapsed() <= Duration::from_secs(2));
    }

    #[tokio::test]
    async fn silent_connection() {
        let backend = TestBackend::start("on", Mode::Silent).await;
        let agent = Agent::start(&backend.url(), 600);
        agent.wait_for(&[ON], SOON).await;
        // Silence limit 2 s, retry 1 s: a new connection by about 3 s.
        tokio::time::sleep(Duration::from_millis(3500)).await;
        assert!(
            backend.connections() >= 2,
            "connections: {}",
            backend.connections()
        );
        assert_eq!(
            agent.calls(),
            [ON, ON],
            "each connection applies its first decision"
        );
    }
}

mod fail_safe_without_the_backend {
    use super::*;

    #[tokio::test]
    async fn backend_unreachable() {
        let backend = TestBackend::unreachable().await;
        let agent = Agent::start(&backend.url(), 2);
        tokio::time::sleep(Duration::from_secs(3)).await;
        agent.wait_for(&[OFF], Duration::from_secs(1)).await;
        // Once, not on every retry.
        tokio::time::sleep(Duration::from_secs(3)).await;
        assert_eq!(agent.calls(), [OFF]);
    }

    #[tokio::test]
    async fn backend_back_after_the_fail_safe() {
        let mut backend = TestBackend::unreachable().await;
        let agent = Agent::start(&backend.url(), 2);
        agent.wait_for(&[OFF], Duration::from_secs(4)).await;

        backend.restart("on").await;
        agent.wait_for(&[OFF, ON], SOON).await;
    }
}

// Spec: display-agent, "Stop after repeated failed switches".
mod stop_after_repeated_failed_switches {
    use super::*;

    #[tokio::test]
    async fn stale_sway_session() {
        let backend = TestBackend::start("on", Mode::Normal).await;
        let mut agent = Agent::start_with(&backend.url(), 600, Swaymsg::Fails);
        agent.wait_for(&[ON], SOON).await;
        backend.set("off");
        agent.wait_for(&[ON, OFF], SOON).await;
        backend.set("on");

        let status = agent.exit_status(SOON).await.expect("the agent stops");
        assert_eq!(status.code(), Some(1));
        assert_eq!(agent.calls(), [ON, OFF, ON]);
    }

    #[tokio::test]
    async fn occasional_failure() {
        let backend = TestBackend::start("on", Mode::Normal).await;
        let mut agent = Agent::start_with(&backend.url(), 600, Swaymsg::FailsOnce);
        agent.wait_for(&[ON], SOON).await;
        backend.set("off");
        agent.wait_for(&[ON, OFF], SOON).await;
        backend.set("on");
        agent.wait_for(&[ON, OFF, ON], SOON).await;
        backend.set("off");
        agent.wait_for(&[ON, OFF, ON, OFF], SOON).await;

        assert_eq!(
            agent.exit_status(Duration::from_millis(500)).await,
            None,
            "keeps running"
        );
    }
}

// Spec: display-agent, "Stay idle while waiting".
mod stay_idle_while_waiting {
    use super::*;

    #[tokio::test]
    async fn quiet_connection() {
        let backend = TestBackend::start("on", Mode::Silent).await;
        let agent = Agent::start_counting_sleeps(&backend.url(), 60);
        agent.wait_for(&[ON], SOON).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
        // Each check of the silence limit ends in a sleep.
        let checks = agent.sleeps();
        assert!(
            checks <= 2,
            "checked the silence limit {checks} times in 5 s"
        );
        assert_eq!(backend.connections(), 1, "the connection stays open");
    }

    // "Silence limit still applies": reconnect_to_the_backend::silent_connection.
}
