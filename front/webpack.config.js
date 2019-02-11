const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const path = require('path');

module.exports = (env, argv) => ({
  mode: argv.mode || 'development',
  output: {
    path: path.join(__dirname, '../assets'),
    filename: '[name].js'
  },
  module: {
    rules: [
      {
        test: /\.scss$/,
				use: [
					'style-loader',
					MiniCssExtractPlugin.loader,
					{ loader: 'css-loader', options: { importLoaders: 1 } },
					{
						loader: "postcss-loader",
						options: {
							minimize: true,
							plugins: [
								require('cssnano')({ preset: 'default' })
							]
						}
					},
					"sass-loader",
        ]
      }
    ]
  },
  plugins: [
    new MiniCssExtractPlugin()
  ]
})
