import test from 'ava'

import { install } from '../index'

test('install function', (t) => {
  t.is(install(), undefined)
})
