<template>
<b-modal centered
				 title="Add a new article"
				 size="lg"
				 @show="reset"
				 @hidden="reset"
				 ref="modal">
  <template v-slot:default>
	  <b-form>
		  <b-form-group label="Title">
			  <b-form-input
				  v-model="title"
				  placeholder="Title"
				  size="lg"
				  :state="validTitle"></b-form-input>
			  <b-form-invalid-feedback :state="validTitle">
				  The title must be a non-empty string.
			  </b-form-invalid-feedback>
		  </b-form-group>
		  <b-form-group label="Authors">
			  <b-form-input
				  v-model="authorsStr"
				  placeholder="Authors"
				  size="lg"
				  :state="validAuthors"></b-form-input>
			  <b-form-invalid-feedback :state="validAuthors">
				The authors must be comma separeted non-empty strings.
			  </b-form-invalid-feedback>
		  </b-form-group>
		  <b-form-group label="File">
			  <b-form-file
				  v-model="file"
				  :state="validFile"
				  size="lg"
				  placeholder="Choose a file"
				  drop-placeholder="Drop a file here..."></b-form-file>
		  </b-form-group>
	  </b-form>
  </template>

  <template v-slot:modal-footer="footer">
    <b-progress
      class="w-100"
      v-if="submitSize > 0"
      :value="submitted"
      :max="submitSize"
      show-progress></b-progress>
    <b-button
      @click="footer.cancel()"
      variant="secondary">Cancel</b-button>
    <b-button
      @click="submit"
      variant="primary"
      :disabled="submitDisable">Add</b-button>
  </template>
</b-modal>
</template>

<script>
import _ from 'lodash';
import axios from 'axios';

export default {
	data() {
		return {
			title: '',
			authorsStr: '',
			file: null,
      submitted: 0,
      submitSize: 0,
      isSubmitting: false
		}
	},
	computed: {
		authors() {
			return _.map(_.split(this.authorsStr, ","), _.trim)
		},
		validTitle() {
			return !/^\s*$/.test(this.title)
		},
		validAuthors() {
			return this.authors.findIndex(author => author.length == 0) < 0
		},
		validFile() {
			return Boolean(this.file)
		},
		valid() {
			return this.validTitle && this.validAuthors && this.validFile
		},
    submitDisable() {
      return !this.valid || this.isSubmitting || this.$refs.modal.busy || this.$refs.modal.isTransitioning
    }
	},
	methods: {
		reset() {
			this.title = ''
			this.authorsStr = ''
			this.file = null
		},
		show() {
			this.$refs.modal.show()
		},
		submit() {
			console.log("submit()")
      this.isSubmitting = true
      var data = new FormData()
      data.append('title', JSON.stringify(this.title))
      data.append('authors', JSON.stringify(this.authors))
      data.append('file', this.file)
      axios.post("/add", data, {
        onUploadProgress: (event) => {
          this.submitSize = event.total
          this.submitted = event.loaded
        }
      }).then(response => {
      }).catch(error => {
        console.log(error)
      }).finally(() => {
        this.isSubmitting = false
        this.submitted = 0
        this.submitSize = 0
      })
		}
	}
}
</script>

<style lang="scss" scoped>
</style>
