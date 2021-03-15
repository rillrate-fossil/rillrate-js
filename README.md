# rillrate-js

[![npm][npm-badge]][npm-url]

[npm-badge]: https://img.shields.io/npm/v/rillrate.svg?style=flat
[npm-url]: https://www.npmjs.com/package/@rillrate/rillrate

Dynamic tracing system that tends to be real-time.

Node.js bindings.

## How to use

Add it as a dependency to your `node.js` project:

```sh
npm install --save @rillrate/rillrate
```

Import it in your code and install a tracer:

```js
rillrate = require('@rillrate/rillrate')
rillrate.install()
```

Add a metric and use methods to update it:

```js
gauge = new rillrate.Gauge('my.gauge', 0, 100)
gauge.set(55.0)

histogram = new rillrate.Histogram('my.histogram', [10, 20, 50, 100, 200, 500])
histogram.add(128.0)
```
