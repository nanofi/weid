
import xs from 'xstream';
import {div, span, ul, li} from '@cycle/dom';
import isolate from '@cycle/isolate';

function Article(sources) {
  const _ = sources.props.debug();
  return {
    DOM: sources.props.map(({title, authors}) => {
      const authorDoms = authors.map(author => span('.article-author', author));

      return li('.article-item', [
        div('.article-delete', 'Del'),
        div('.article-content', [
          div('.article-title', title),
          div('.article-authors', authorDoms)
        ])
      ]);
    })
  }
}

function ArticleList(sources) {
  return {
    DOM: sources.props.map(({articles}) => {
      const articleDoms = articles.map(article => {
        return isolate(Article)({
          DOM: sources.DOM,
          props: xs.of(article)
        }).DOM;
      });

      return xs.combine(...articleDoms).map(articles => ul('.article-list', articles));
    }).flatten()
  };
}

export default ArticleList;
