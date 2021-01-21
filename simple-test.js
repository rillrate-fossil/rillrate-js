const { install } = require('./index')

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function test() {
  install()
  await sleep(5000)
}

test()
