<template>
  <div class="action-bar">
    <b-form-group>
      <b-input-group>
        <b-form-input
          ref="search"
          v-model="search"
          placeholder="Type search queries"
          size="lg"></b-form-input>

        <b-button v-b-modal.add-modal slot="append" variant="primary">Add</b-button>
      </b-input-group>
    </b-form-group>

    <b-modal id="add-modal"
             title="Add a new article"
             @show="resetAddForm">
    </b-modal>
  </div>
</template>

<script>
import _ from 'lodash';

export default {
  data() {
    return {
      search: ''
    };
  },
  watch: {
    search: function(query) {
      this.querySearch(query)
    }
  },
  methods: {
    querySearch: _.debounce(function(query) {
      // Do search here

      
    }.bind(this), 300),
    resetAddForm() {
      
    },
    focus() {
      this.$refs.search.focus()
    }
  }
}
</script>

<style lang="scss" scoped>
.action-bar {
  margin: 2em 0;
}
</style>
