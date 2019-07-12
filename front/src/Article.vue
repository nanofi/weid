<template>
  <div>
    <h3>{ article.title }</h3>
    <ul class="list-inline">
      <li v-for="(author, index) in article.authors" :key="index">{ author }</li>
    </ul>
  </div>
</template>

<script>
export default {
  props: {
    article: { type: Object, required: true }
  }
}
</script>

<style lang="scss" scoped>

</style>
