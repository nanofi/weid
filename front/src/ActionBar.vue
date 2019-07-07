<template>
<div class="action-bar">
  <b-form-group>
    <b-input-group>
      <b-form-input
        ref="search"
        v-model="search"
        placeholder="Type search queries"
        size="lg"></b-form-input>
			
      <b-button @click="addClick" slot="append" variant="primary">Add</b-button>
    </b-input-group>
  </b-form-group>
	
	<add-modal ref="addModal"></add-modal>
</div>
</template>

<script>
import _ from 'lodash';
import AddModal from './AddModal.vue';

export default {
  data() {
    return {
      search: ''
    };
  },
  watch: {
    search(query) {
      this.querySearch(query)
    }
  },
  methods: {
    querySearch: _.debounce(function(query) {
      // Do search here

      
    }.bind(this), 300),
		addClick() {
			this.$refs.addModal.show()
			this.$refs.addModal.title = this.search
		},
    focus() {
      this.$refs.search.focus()
    }
  },
	components: { AddModal }
}
</script>

<style lang="scss" scoped>
.action-bar {
  margin: 2em 0;
}
</style>
