<template>
  <div class="root">
    <action-bar ref="actionBar"></action-bar>
		
    
  </div>
</template>

<script>
import ActionBar from './ActionBar.vue';

export default {
  data() {
    return {};
  },
  mounted() {
    this.$refs.actionBar.focus();
  },
  components: {
    ActionBar
  }
}
</script>

<style lang="scss" scoped>

.root {
  @include make-container();
  @include make-container-max-widths(); 
}

</style>
