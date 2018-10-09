
function intent(sources) {
}

function model(actions) {
}

function view(state) {
}

function Item(sources) {
  return {
    DOM: view(model(intent(sources)))
  };
}

export default Item;
