import { describe, it, expect } from 'vitest'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

/**
 * Every routed page must have exactly one root element.
 *
 * AppShell renders pages inside <Transition mode="out-in">. Vue applies the
 * leave hooks to the component's root `el`; for a fragment root that is a text
 * anchor rather than an element, so the browser never reports the transition
 * finishing and the *next* page never mounts — the content area goes blank.
 *
 * jsdom does not reproduce it (its getComputedStyle on a text node resolves
 * harmlessly), so this is a structural check rather than a behavioural one.
 */
function countRootNodes(source: string): number {
  const m = source.match(/<template>([\s\S]*)<\/template>/)
  if (!m) return 0
  const body = m[1]
  let depth = 0
  let roots = 0
  const voidTags = new Set(['br', 'img', 'input', 'hr', 'meta', 'link'])
  const re = /<(\/?)([A-Za-z][\w-]*)((?:[^>"']|"[^"]*"|'[^']*')*?)(\/?)>/g
  let tag: RegExpExecArray | null
  while ((tag = re.exec(body))) {
    const [, closing, name, , selfClose] = tag
    if (closing) {
      depth -= 1
    } else if (selfClose || voidTags.has(name.toLowerCase())) {
      if (depth === 0) roots += 1
    } else {
      if (depth === 0) roots += 1
      depth += 1
    }
  }
  return roots
}

const pagesDir = join(__dirname, '..', 'pages')
const pages = readdirSync(pagesDir).filter((f) => f.endsWith('.vue'))

describe('routed pages', () => {
  it('there are pages to check', () => {
    expect(pages.length).toBeGreaterThan(0)
  })

  for (const file of pages) {
    it(`${file} has exactly one root element`, () => {
      const source = readFileSync(join(pagesDir, file), 'utf8')
      const roots = countRootNodes(source)
      expect(
        roots,
        `${file} renders ${roots} root nodes. Wrap the template in a single element — ` +
          `a fragment root breaks <Transition mode="out-in"> and blanks the next page.`,
      ).toBe(1)
    })
  }
})
