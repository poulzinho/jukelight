import type { Player } from '../api'

type Entry = NonNullable<Player['queue']>[number]

/** A row of "Up next": a group's heading, or an entry. */
export type Row =
  | { kind: 'heading'; text: string; key: string; /** The key of the entry it heads. */ first: string }
  | { kind: 'entry'; key: string; title: string; artist: string | null; artUrl: string | null }

/** A title's beginning and the rest, split at its last ": " or " - ". */
function split(title: string): { work: string; rest: string } | null {
  const colon = title.lastIndexOf(': ')
  const dash = title.lastIndexOf(' - ')
  const at = Math.max(colon, dash)
  if (at <= 0) return null
  return { work: title.slice(0, at), rest: title.slice(at + (at === colon ? 2 : 3)) }
}

/**
 * Each entry keyed by its title and artists, so an entry keeps its element
 * as the queue moves on; a repeated entry gets a number.
 */
function keys(entries: Entry[]): string[] {
  const seen = new Map<string, number>()
  return entries.map((entry) => {
    const name = `${entry.title}|${entry.artists.join(', ')}`
    const count = seen.get(name) ?? 0
    seen.set(name, count + 1)
    return `${name}|${count}`
  })
}

interface Group {
  work: string | null
  artist: string | null
  members: { entry: Entry; key: string; rest: string }[]
}

/**
 * "Up next" as rows. Consecutive entries of one work (the beginning of a
 * title before its last ": " or " - ", shared with a neighbour, or by the
 * first entry with the current track) share the work as a heading and show
 * the rest of their titles; the first names its artists, later ones only
 * when the artists change. Otherwise consecutive entries with the same main
 * artist share that artist as a heading, with any others as "with …".
 * Single entries show their title and artists. A group starts again after
 * anything in between.
 */
export function groupQueue(entries: Entry[], current: { title: string; artists: string[] }): Row[] {
  const parts = entries.map((entry) => split(entry.title))
  const currentWork = split(current.title)?.work ?? null
  const works = parts.map((part, i) => {
    const work = part?.work ?? null
    if (!work) return null
    const shared = parts[i - 1]?.work === work || parts[i + 1]?.work === work || (i === 0 && currentWork === work)
    return shared ? work : null
  })
  const entryKeys = keys(entries)

  const groups: Group[] = []
  entries.forEach((entry, i) => {
    const work = works[i] ?? null
    const main = entry.artists[0] ?? null
    const key = entryKeys[i] ?? String(i)
    const rest = work ? (parts[i]?.rest ?? entry.title) : entry.title
    const last = groups[groups.length - 1]
    if (last && work && last.work === work) last.members.push({ entry, key, rest })
    else if (last && !work && !last.work && main && last.artist === main) last.members.push({ entry, key, rest })
    else groups.push({ work, artist: work ? null : main, members: [{ entry, key, rest }] })
  })

  const rows: Row[] = []
  for (const group of groups) {
    const first = group.members[0]
    if (!first) continue
    if (group.work) {
      rows.push({ kind: 'heading', text: group.work, key: `work|${first.key}`, first: first.key })
      let named: string | null = null
      for (const { entry, key, rest } of group.members) {
        const artists = entry.artists.join(', ')
        rows.push({ kind: 'entry', key, title: rest, artist: artists && artists !== named ? artists : null, artUrl: entry.artUrl })
        named = artists
      }
    } else if (group.members.length > 1 && group.artist) {
      rows.push({ kind: 'heading', text: group.artist, key: `artist|${first.key}`, first: first.key })
      for (const { entry, key } of group.members) {
        const others = entry.artists.slice(1)
        rows.push({ kind: 'entry', key, title: entry.title, artist: others.length ? `with ${others.join(', ')}` : null, artUrl: entry.artUrl })
      }
    } else {
      const { entry, key } = first
      rows.push({ kind: 'entry', key, title: entry.title, artist: entry.artists.join(', ') || null, artUrl: entry.artUrl })
    }
  }
  return rows
}
