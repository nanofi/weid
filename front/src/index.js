import Vue from 'vue';
import BootstrapVue from 'bootstrap-vue';

Vue.use(BootstrapVue);

import App from './App.vue';

import './index.scss';

new Vue({
  el: '#app',
  template: '<App/>',
  components: { App },
});

