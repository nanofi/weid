const VueLoaderPlugin = require('vue-loader/lib/plugin');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const path = require('path');

module.exports = ( env, argv ) => ({
  output: {
    path: path.join(__dirname, '../assets'),
    filename: '[name].js'
  },
  resolve: {
    alias: {
      'vue$': 'vue/dist/vue.esm.js'
    }
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: 'babel-loader'
      },
      {
        test: /\.vue$/,
        loader: 'vue-loader'
      },
      {
        test: /\.(sa|sc|c)ss$/,
				use: [
          MiniCssExtractPlugin.loader,
					{
            loader: 'css-loader',
            options: {
              importLoaders: 1,
            }
          },
					{
						loader: "postcss-loader",
						options: {
							minimize: true,
							plugins: [
                require('autoprefixer'),
								require('cssnano')({ preset: 'default' })
							]
						}
					},
					{
            loader: "sass-loader",
            options: {
              data: '@import "./base";',
              includePaths: [path.resolve(__dirname, './src/')]
            }
          }
        ]
      }
    ]
  },
  plugins: [
    new VueLoaderPlugin(),
    new MiniCssExtractPlugin()
  ]
});
