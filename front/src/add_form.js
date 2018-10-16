
import xs from 'xstream';
import sampleCombine from 'xstream/extra/sampleCombine'
import {div, span, h1, h2, input, label} from '@cycle/dom';
import isolate from '@cycle/isolate';

function AddFileInput(sources) {
  const id = '_' + Math.random().toString(36).substr(2, 9);

  const file = sources.DOM.select('.add-file')
    .events('input')
    .map(ev => ev.target.files[0])
    .startWith(undefined);

  const path = file.map(file => {
    if(file === undefined) {
      return "Select a file";
    } else {
      return file.name;
    }
  });

  return {
    DOM: path.map(path => label('.add-file-label', {attrs: {for: id}}, [
      path,
      input('.add-file', {attrs: {id: id, type: 'file'}})
    ])),
    file: file,
  }
}

function AddForm(sources) {
  const addClick = sources.DOM
    .select('.add')
    .events('click');
  const addResponse = sources.HTTP
    .select('add')
    .flatten();

  const addFileInput = AddFileInput({
    DOM: sources.DOM,
  });

  const title = sources.DOM.select('input.add-title')
    .events('input')
    .map(ev => ev.target.value)
    .startWith('');
  const authors = sources.DOM.select('input.add-title')
    .events('input')
    .map(ev => ev.target.value.split(',').map(name => name.trim()))
    .startWith([]);
  const file = addFileInput.file
    .filter(file => file !== undefined)
    .map(file => xs.create({
      reader: new FileReader(),
      start: function(listener) {
        this.reader.readAsDataURL(file);
        this.reader.onload = () => {
          listener.next(this.reader.result);
          listener.complete();
        };
        this.reader.onerror = (err) => {
          listener.error(err);
        };
      },
      stop: function() {
        this.reader.onload = this.reader.onerror = undefined;
      }
    })).flatten();

  const formValues = xs.combine(title, authors, file).map(([title, authors, file]) => ({
    title: title,
    authors: authors,
    file: file,
  }));

  const addWithValues = addClick
    .compose(sampleCombine(formValues))
    .map(([_, values]) => values);

  const enable = xs.merge(addWithValues.mapTo(false), addResponse.mapTo(true)).startWith(true);


  const state = xs.combine(addFileInput.DOM, enable).map(arr => ({
    addFileInput: arr[0],
    enable: arr[1],
  }));

  return {
    DOM: state.map(({addFileInput, enable}) => {
      const disabled = enable ? '' : '.disabled';

      return div('.add-form', [
        h1('Add new article'),
        div('.add-field' + disabled, [
          h2('Title'),
          input('.add-title', {attrs: {type: 'text', placeholder: 'Title'}}),
        ]),
        div('.add-field' + disabled, [
          h2('Authors (comma separated)'),
          input('.add-authors', {attrs: {type: 'text', placeholder: 'Authors'}}),
        ]),
        div('.add-field' + disabled, [
          h2('File'),
          addFileInput
        ]),
        div('.add' + disabled, 'Add'),
      ]);
    }),
    HTTP: addWithValues.map(values => ({
      url: '/add',
      category: 'add',
      method: 'POST',
      send: values
    })),
    response: addResponse,
  };
}

export default AddForm;
