// Spec: kiosk-display, "Up next": entries grouped by work or by artist.
import { expect, test } from 'vitest'
import { type Row, groupQueue } from './group'

const entry = (title: string, ...artists: string[]) => ({ title, artists, artUrl: null })
const current = (title: string, ...artists: string[]) => ({ title, artists })

/** The rows as plain text: "# heading", "title · artist" or "title". */
function rows(list: Row[]): string[] {
  return list.map((row) => (row.kind === 'heading' ? `# ${row.text}` : row.artist ? `${row.title} · ${row.artist}` : row.title))
}

test('movements of one work: the work as a heading, the first entry names the artist', () => {
  const list = groupQueue(
    [entry('Example Composer: Example Concerto, Op. 1: II. Andante', 'Example Violinist'), entry('Example Composer: Example Concerto, Op. 1: III. Presto', 'Example Violinist')],
    current('Example Composer: Example Concerto, Op. 1: I. Allegro', 'Example Violinist'),
  )
  expect(rows(list)).toEqual(['# Example Composer: Example Concerto, Op. 1', 'II. Andante · Example Violinist', 'III. Presto'])
})

test('the first entry continues the current work, alone in the queue', () => {
  const list = groupQueue([entry('Example Concerto: II. Andante', 'Example Violinist'), entry('Terca', 'Example Singer')], current('Example Concerto: I. Allegro', 'Example Violinist'))
  expect(rows(list)).toEqual(['# Example Concerto', 'II. Andante · Example Violinist', 'Terca · Example Singer'])
})

test('performer changes within a work: the artist shows where it changes', () => {
  const list = groupQueue(
    [
      entry('Goldberg Variations, BWV 988: Variation 1', 'Example Pianist'),
      entry('Goldberg Variations, BWV 988: Variation 2', 'Another Example Pianist'),
      entry('Goldberg Variations, BWV 988: Variation 3', 'Another Example Pianist'),
    ],
    current('Example Song', 'Example Singer'),
  )
  expect(rows(list)).toEqual([
    '# Goldberg Variations, BWV 988',
    'Variation 1 · Example Pianist',
    'Variation 2 · Another Example Pianist',
    'Variation 3',
  ])
})

test('a work without the composer in the title', () => {
  const list = groupQueue(
    [entry('Symphony No. 5 in C Minor, Op. 67: II. Andante con moto', 'Example Orchestra'), entry('Symphony No. 5 in C Minor, Op. 67: III. Allegro', 'Example Orchestra')],
    current('Example Song', 'Example Singer'),
  )
  expect(rows(list)[0]).toBe('# Symphony No. 5 in C Minor, Op. 67')
})

test('a dash as the separator', () => {
  const list = groupQueue(
    [entry('Piano Sonata No. 14 - II. Allegretto', 'Example Pianist'), entry('Piano Sonata No. 14 - III. Presto agitato', 'Example Pianist')],
    current('Example Song', 'Example Singer'),
  )
  expect(rows(list)).toEqual(['# Piano Sonata No. 14', 'II. Allegretto · Example Pianist', 'III. Presto agitato'])
})

test('a lone title with a separator stays whole', () => {
  const list = groupQueue([entry('Interlude: The Long Way Home', 'Example Singer'), entry('Terca', 'Example Band')], current('Example Song', 'Example Singer'))
  expect(rows(list)).toEqual(['Interlude: The Long Way Home · Example Singer', 'Terca · Example Band'])
})

test('an album by one artist: the artist as a heading', () => {
  const list = groupQueue(
    [entry('Example Song A', 'Example Singer'), entry('Example Song B', 'Example Singer'), entry('Example Song C', 'Example Singer')],
    current('Example Song', 'Example Singer'),
  )
  expect(rows(list)).toEqual(['# Example Singer', 'Example Song A', 'Example Song B', 'Example Song C'])
})

test('a featured artist stays in the group, "with" the guest', () => {
  const list = groupQueue([entry('Example Song A', 'Example Singer'), entry('Example Song B', 'Example Singer', 'Guest Artist')], current('Example Song', 'Example Singer'))
  expect(rows(list)).toEqual(['# Example Singer', 'Example Song A', 'Example Song B · with Guest Artist'])
})

test('an artist interrupted: the heading again for the second pair', () => {
  const list = groupQueue(
    [entry('A', 'Example Singer'), entry('B', 'Example Singer'), entry('C', 'Example Band'), entry('D', 'Example Singer'), entry('E', 'Example Singer')],
    current('Example Song', 'Example Singer'),
  )
  expect(rows(list)).toEqual(['# Example Singer', 'A', 'B', 'C · Example Band', '# Example Singer', 'D', 'E'])
})

test('no artist: not grouped, titles only', () => {
  const list = groupQueue([entry('Unknown Track 01'), entry('Unknown Track 02')], current('Example Song', 'Example Singer'))
  expect(rows(list)).toEqual(['Unknown Track 01', 'Unknown Track 02'])
})

test('a single entry: title and artists, as before', () => {
  const list = groupQueue([entry('Alfonsina y el Mar', 'Example Singer')], current('Example Song', 'Example Singer'))
  expect(rows(list)).toEqual(['Alfonsina y el Mar · Example Singer'])
})

test('every entry keeps a stable key; a heading belongs to its first entry', () => {
  const list = groupQueue([entry('A', 'Example Singer'), entry('B', 'Example Singer'), entry('A', 'Example Singer')], current('Example Song', 'Example Singer'))
  const entries = list.filter((row) => row.kind === 'entry')
  expect(new Set(entries.map((row) => row.key)).size).toBe(3)
  const heading = list.find((row) => row.kind === 'heading')
  expect(heading?.kind === 'heading' && heading.first).toBe(entries[0]?.key)
})
