
import xs from 'xstream';
import {div, span, ul, li} from '@cycle/dom';
import isolate from '@cycle/isolate';
import sampleCombine from 'xstream/extra/sampleCombine';

function Article(sources) {
  const del = sources.DOM
    .select('.article-delete')
    .events('click');
  const click = sources.DOM
    .select('.article-content')
    .events('click').debug();
  const id = sources.props.map(({id}) => id);

  const delReq = del.compose(sampleCombine(id)).map(([del, id]) => id);
  const clickEv = click.compose(sampleCombine(id)).map(([ev, id]) => id);

  clickEv.addListener({ next: id => {
    console.log(id);
    window.location.href = `/view/${id}`;
  }});
  
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
    }),
    HTTP: delReq.map(id => ({
      url: `/delete/${id}`,
      category: 'delete',
      method: 'DELETE'
    }))
  };
}

function ArticleList(sources) {
  const articles = sources.props.map(({articles}) => articles.map(article => isolate(Article)({
    DOM: sources.DOM,
    props: xs.of(article)
  })));
  const response = sources.HTTP.select('delete').flatten().map(res => res.body);
  
  return {
    DOM: articles.map(articles => xs.combine(...articles.map(article => article.DOM)).map(articles => ul('.article-list', articles))).flatten(),
    HTTP: articles.map(articles => xs.merge(...articles.map(article => article.HTTP))).flatten(),
    response: response
  };
}

export default ArticleList;
