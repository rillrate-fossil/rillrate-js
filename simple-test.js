const { install, Counter, Gauge, Pulse, Dict, Logger, Histogram } = require('./index')

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms))
}

async function test() {
    install()

    counter = new Counter("my.counter");
    counter.inc(1.2);

    gauge = new Gauge("my.gauge", 0, 100);
    gauge.set(20);

    pulse = new Pulse("my.pulse");
    pulse.set(1.2);

    histogram = new Histogram("my.histogram", [10, 50, 100, 200, 500, 1000]);
    histogram.add(150);

    dict = new Dict("my.dict");
    dict.set("key", "value");

    logger = new Logger("my.logger");
    logger.log("log message");

    await sleep(5000)
}

test()
