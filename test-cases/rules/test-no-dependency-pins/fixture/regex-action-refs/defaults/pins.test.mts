expect(ref).toMatch(/vendor\/sample@v4/)
expect(ref).toBe('vendor/sample@v4')
expect(ref).toMatch(/@v\d+/)
expect(ref).toMatch(/^vendor\/sample@v4$/)
expect(ref).toMatch(/^vendor\/sample@v1\.2\.3$/i)
expect(ref).toMatch(/uses: vendor\/sample@v4/)
expect(ref).toMatch(/vendor\/sample@v(4|5)/)
expect(ref).toMatch(/vendor\/sample@v[45]/)
expect(ref).toMatch(/vendor\/sample@v4|v5/)
expect(ref).toMatch(/vendor\/sample@v4?/)
expect(ref).toMatch(/vendor\/sample@v4\d+/)
expect(ref).toMatch(/vendor\/sample@v1.2.3/)
expect(ref).toMatch(/vendor\/sample@v0\.0\.0/)
expect(ref).toMatch(/vendor\/sample@v0/)
expect(ref).toMatch(/vendor\x2fsample@v4/)
expect(ref).toMatch(/vendor\/sample@v4{1,2}/)
expect(ref).toMatch(/(?:vendor\/sample@v4)/)
expect(ref).toMatch(/.*vendor\/sample@v4.*/)
expect(ref).toMatch(/vendor\/sample@v4\b/)
expect(ref).toMatch(/vendor\/sample@v4\$/)
const text = '/vendor\\/sample@v4/'
const template = `/vendor\\/sample@v4/`
// expect(ref).toMatch(/vendor\/sample@v4/)
/* expect(ref).toMatch(/vendor\/sample@v4/) */
expect(ref).toMatch(
  /vendor\/sample@v4/,
)
expect(ref).toMatch(/^vendor\/sample@$/)
