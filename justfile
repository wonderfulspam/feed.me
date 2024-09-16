set quiet

spacefeeder_config_path := "spacefeeder.toml"

add_feed slug url author tier="new": (_check_binary_exists "dasel")
  dasel put -f {{spacefeeder_config_path}} -r toml -t string -v '{{url}}' 'feeds.{{slug}}.url' && \
  dasel put -f {{spacefeeder_config_path}} -r toml -t string -v '{{author}}' 'feeds.{{slug}}.author' && \
  dasel put -f {{spacefeeder_config_path}} -r toml -t string -v '{{tier}}' 'feeds.{{slug}}.tier'

fetch_feeds: build_spacefeeder
  spacefeeder fetch

[no-exit-message]
find_feed base_url: build_spacefeeder
  spacefeeder find-feed --base-url {{base_url}}

build: fetch_feeds
  zola build

build_spacefeeder:
  echo "Building spacefeeder"
  cd spacefeeder && cargo install --quiet --path . --locked

serve:
  zola serve

publish_to_netlify: build
  zip -r site.zip public
  curl -H "Content-Type: application/zip" \
     -H "Authorization: Bearer $NETLIFY_DEPLOY_TOKEN" \
     --data-binary "@site.zip" \
     https://api.netlify.com/api/v1/sites/feed-me-feeds.netlify.com/deploys
  rm site.zip

[no-exit-message]
_check_binary_exists binary_name:
  command -v {{binary_name}} > /dev/null 2>&1 || { echo "{{binary_name}} not found, aborting"; exit 1; }
