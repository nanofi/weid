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
  const addClick = sources.DOM
    .select('.add-button')
    .events('click')
    .mapTo(true);
  const closeClick = sources.DOM
    .select('.actions-close')
    .events('click')
    .mapTo(false)
  const actionsShow = xs.merge(addClick, closeClick)
    .startWith(false);

  const articleList = ArticleList({
    DOM: sources.DOM,
    props: searchResponse.map(response => ({
      articles: response
    }))
  });

  const addForm = AddForm({
    DOM: sources.DOM,
    HTTP: sources.HTTP,
  });

  const searchRequest = xs.combine(search, addForm.response.startWith({}))
    .map(([search, add]) => search)
    .map(search => ({
    url: `/search?q=${encodeURI(search)}`,
    category: 'search'
  }));

  const requests = xs.merge(searchRequest, addForm.HTTP);

  const state = xs.combine(articleList.DOM, addForm.DOM, actionsShow).map(([articleListDom, addFormDom, actionsShow]) => ({
    addForm: addFormDom,
    articleList: articleListDom,
    actionsShow: actionsShow,
  }));

  return {
    DOM: state.map(state => {
      const actionClass = state.actionsShow ? '.actions.actions-show' : '.actions';

      return div('.main', [
        div(actionClass, [
          div('.actions-close.siimple-close'),
          state.addForm
        ]),
        input('.search', {attrs: {type: 'text', placeholder: 'Search...'}}),
        div('.add-button', 'Add new article'),
        state.articleList
      ]);
    }),
    HTTP: requests,
  };
}

const drivers = {
  DOM: makeDOMDriver('#app'),
  HTTP: makeHTTPDriver(),
  Time: timeDriver
};

run(main, drivers);
