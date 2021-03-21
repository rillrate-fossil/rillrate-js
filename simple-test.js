const { install, Counter, Gauge, Pulse, Dict, Logger, Histogram, Table } = require('./index')

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

    table = new Table("my.table", [[0, "Column 1"], [1, "Column 2"]]);
    table.add_row(0);
    table.add_row(1);
    table.set_cell(0, 0, "Cell of Row 1");
    table.set_cell(1, 0, "Cell of Row 2");
    table.set_cell(1, 1, "has value");

    await sleep(150000)
}

test()
