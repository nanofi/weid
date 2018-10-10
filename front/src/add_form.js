
import xs from 'xstream';
import {div, span, h1, h2, input, label} from '@cycle/dom';
import isolate from '@cycle/isolate';

function AddFileInput(sources) {
  const id = '_' + Math.random().toString(36).substr(2, 9);

  const file = sources.DOM.select('.add-file')
    .events('input')
    .map(ev => ev.target.files[0])
    .startWith(null);

  const path = file.map(file => {
    if(file === null) {
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

  const addFileInput = AddFileInput({
    DOM: sources.DOM,
  });

  const state = addFileInput.DOM.map(addFileInputDom => ({
    addFileInput: addFileInputDom
  }));

  return {
    DOM: state.map(({addFileInput}) => div('.add-form', [
      h1('Add new article'),
      div('.add-field', [
        h2('Title'),
        input('.add-title', {attrs: {type: 'text', placeholder: 'Title'}}),
      ]),
      div('.add-field', [
        h2('Authors (comma separated)'),
        input('.add-authors', {attrs: {type: 'text', placeholder: 'Authors'}}),
      ]),
      div('.add-field', [
        h2('File'),
        addFileInput
      ]),
      div('.add', 'Add'),
    ])),
    title: sources.DOM.select('input.add-title')
      .events('input')
      .map(ev => ev.target.value).debug(),
    authors: sources.DOM.select('input.add-title')
      .events('input')
      .map(ev => ev.target.value.split(',').map(name => name.trim())).debug(),
    file: addFileInput.file
  };
}

export default AddForm;
