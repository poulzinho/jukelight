import { Kiosk } from './Kiosk'
import { usePlayer } from './usePlayer'

export default function App() {
  const { player, connection, clockOffsetMs } = usePlayer()
  return <Kiosk player={player} connection={connection} clockOffsetMs={clockOffsetMs} />
}
