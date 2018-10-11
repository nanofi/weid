import './index.scss';

import xs from 'xstream';
import {run} from '@cycle/run';
import {makeDOMDriver, div, input, p, h1, h2} from '@cycle/dom';
import {makeHTTPDriver} from '@cycle/http';
import {timeDriver} from '@cycle/time';

import ArticleList from './article_list';
import AddForm from './add_form';

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

  const addForm = AddForm({
    DOM: sources.DOM,
    props: xs.of(null)
  })

  const state = xs.combine(articleList.DOM, addForm.DOM).map(([articleListDom, addFormDom]) => ({
    addForm: addFormDom,
    articleList: articleListDom,
  }));

  return {
    DOM: state.map(({articleList, addForm}) => {
      return div('.main', [
        div('.actions', [
          div('.siimple-close'),
          addForm
        ]),
        input('.search', {attrs: {type: 'text', placeholder: 'Search...'}}),
        div('.add-button', 'Add new article'),
        articleList
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
