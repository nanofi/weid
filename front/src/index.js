import './index.scss';

import xs from 'xstream';
import {run} from '@cycle/run';
import {makeDOMDriver, div, input, p} from '@cycle/dom';

function main(sources) {
  const sinks = {
    DOM: sources.DOM.select('input').events('click')
      .map(ev => ev.target.checked)
      .startWith(false)
      .map(toggled =>
        div([
          input({attrs: {type: 'checkbox'}}), 'Toggle me',
          p(toggled ? 'ON' : 'off')
        ])
      )
  };
  return sinks;
}

const drivers = {
  DOM: makeDOMDriver('#app')
};

run(main, drivers);
