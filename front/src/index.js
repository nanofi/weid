import './index.scss';

import xs from 'xstream';
import {run} from '@cycle/run';
import {makeDOMDriver, div, input, p, h1, h2} from '@cycle/dom';
import {makeHTTPDriver} from '@cycle/http';
import {timeDriver} from '@cycle/time';

import ArticleList from './article_list';


function main(sources) {
  const search = sources.DOM
    .select('input.search')
    .events('input')
    .compose(sources.Time.debounce(300))
    .map(ev => ev.target.value)
    .startWith('');
  const searchResponse = sources.HTTP
    .select('search')
    .flatten()
    .map(res => res.body)
    .startWith([]);

  const articleList = ArticleList({
    DOM: sources.DOM,
    props: searchResponse.map(response => ({
      articles: response
    }))
  });

  return {
    DOM: articleList.DOM.map(articleListDom => {
      return div([
        div('.actions', [
          h1('Add new article'),
          div('.add-field', [
            h2('Title'),
            input('.add-title', {attrs: {type: 'text', placeholder: 'Title'}}),
          ]),
          div('.add-field', [
            h2('Authors (comma separated)'),
            input('.add-authors', {attrs: {type: 'text', placeholder: 'Authors'}}),
          ]),
          div('.add', 'Add'),
        ]),
        input('.search', {attrs: {type: 'text', placeholder: 'Search...'}}),
        articleListDom
      ]);
    }),
    HTTP: search.map(search => ({
      url: `/search?q=${encodeURI(search)}`,
      category: 'search'
    }))
  };
}

const drivers = {
  DOM: makeDOMDriver('#app'),
  HTTP: makeHTTPDriver(),
  Time: timeDriver
};

run(main, drivers);
