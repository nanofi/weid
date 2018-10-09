import './index.scss';

import xs from 'xstream';
import debounce from 'xstream/extra/debounce'
import {run} from '@cycle/run';
import {makeDOMDriver, div, input, p, h1} from '@cycle/dom';
import {makeHTTPDriver} from '@cycle/http';
import {timeDriver} from '@cycle/time';

function intent(sources) {
  return {
    responseSearch: sources.HTTP.select('search')
      .flatten()
      .map(res => res.body)
      .debug()
  };
}

function model(actions) {
  const searchResponse = actions.responseSearch.startWith([]);

  return xs.combine(searchResponse)
    .map(([searchResponse]) => {
      return {searchResponse};
    });
}

function view(state) {
  return state.map(({searchResponse}) =>
    div([
      input('.search', {attrs: {type: 'text', placeholder: 'Search...'}}),
      searchResponse
    ])
  );
}

function intentHTTP(sources) {
  return {
    inputSearch: sources.DOM.select('input.search').events('input')
      .compose(sources.Time.debounce(300))
      .map(ev => ev.target.value)
  };
}

function modelHTTP(actions) {
  const search = actions.inputSearch.startWith('');

  return xs.combine(search)
    .map(([search]) => {
      return {search};
    });

}

function request(state) {
  return state.map(({search}) => ({
    url: `/search?q=${encodeURI(search)}`,
    category: 'search'
  }));
}

function main(sources) {
  return {
    DOM: view(model(intent(sources))),
    HTTP: request(modelHTTP(intentHTTP(sources)))
  };
}

const drivers = {
  DOM: makeDOMDriver('#app'),
  HTTP: makeHTTPDriver(),
  Time: timeDriver
};

run(main, drivers);
