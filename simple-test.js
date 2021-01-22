const { install, Gauge } = require('./index')

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms))
}

async function test() {
    install()
    gauge = new Gauge("my.provider");
    gauge.set(1.2);
    await sleep(5000)
}

test()
