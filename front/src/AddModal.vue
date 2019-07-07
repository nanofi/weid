<template>
<b-modal centered
				 title="Add a new article"
				 ok-title="Add"
				 :ok-disabled="!valid"
				 size="lg"
				 @show="reset"
				 @hidden="reset"
				 @ok.prevent="submit"
				 ref="modal">
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
</b-modal>
</template>

<script>
import _ from 'lodash';

export default {
	data() {
		return {
			title: '',
			authorsStr: '',
			file: null
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
			
		}
	}
}
</script>

<style lang="scss" scoped>
</style>
