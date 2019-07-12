<template>
  <div class="root">
    <action-bar ref="actionBar"
      @search="search"
      @added="added"></action-bar>
		
    <article v-for="article in articles" :article="article" :key="article.id"></article>
  </div>
</template>

<script>
import axios from 'axios';

import ActionBar from './ActionBar.vue';
import Article from './Article.vue'

export default {
  data() {
    return {
      query: '',
      articles: [],
      isSearching: false,
    };
  },
  mounted() {
    this.$refs.actionBar.focus()
    this.doSearch()
  },
  methods: {
    search(query) {
      this.query = query
      this.doSearch()
    },
    added(article) {
      this.doSearch()
    },
    doSearch() {
      this.isSearching = true
      axios.get('/search', {
        params: { q: this.query }
      }).then(response => {
        this.articles = response.data
      }).catch(error => {
				this.$bvModal.msgBoxOk(`Failed to search: ${error.response.data}`, {
					title: 'Error!',
					centered: true,
				})
      }).finally(() => {
        this.isSearching = false
      })
    }
  },
  components: {
    ActionBar,
    Article,
  }
}
</script>

<style lang="scss" scoped>

.root {
  @include make-container();
  @include make-container-max-widths(); 
}

</style>
