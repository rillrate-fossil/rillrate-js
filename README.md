# rillrate-js

Dynamic tracing system that tends to be real-time.

Node.js bindings.

## How to use

Add it as a dependency to your `node.js` project:

```sh
npm install --save @rillrate/rillrate
```

Import it in your code and install a tracer:

```js
rillrate = require('@rillrate/rillrate');
rillrate.install();
```

Add a metric and use methods to update it:

```js
gauge = new rillrate.Gauge("my.gauge");
gauge.set(123.0);
```
