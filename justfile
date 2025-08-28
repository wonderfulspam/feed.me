set quiet

spacefeeder_config_path := "spacefeeder.toml"

init: build_spacefeeder
  spacefeeder init

add_feed slug url author tier="new": build_spacefeeder
  spacefeeder add-feed --slug "{{slug}}" --url "{{url}}" --author "{{author}}" --tier "{{tier}}"

export_feeds: build_spacefeeder
  spacefeeder export

import_feeds input_path tier="new": build_spacefeeder
  spacefeeder import --input-path "{{input_path}}" --tier "{{tier}}"

fetch_feeds: build_spacefeeder
  spacefeeder fetch

search query *filters="": build_spacefeeder
  spacefeeder search "{{query}}" {{filters}}

analyze_tags format="summary":
  python3 tools/analyze_tags.py . --format "{{format}}"

analyze_feeds:
  cargo run --manifest-path spacefeeder/Cargo.toml -- analyze-feeds

[no-exit-message]
find_feed base_url: build_spacefeeder
  spacefeeder find-feed --base-url {{base_url}}

build: build_spacefeeder
  spacefeeder build

build_spacefeeder:
  echo "Building spacefeeder"
  cd spacefeeder && cargo install --quiet --path . --locked

serve: build_spacefeeder
  spacefeeder serve

publish_to_netlify: build
  zip -r site.zip public
  curl -H "Content-Type: application/zip" \
     -H "Authorization: Bearer $NETLIFY_DEPLOY_TOKEN" \
     --data-binary "@site.zip" \
     https://api.netlify.com/api/v1/sites/feed-me-feeds.netlify.com/deploys
  rm site.zip

# Release automation using cargo-release
# Install with: cargo install cargo-release
release version="patch":
  cd spacefeeder && cargo release version {{version}} --execute
  cd spacefeeder && cargo release replace --execute
  cd spacefeeder && cargo build --quiet
  echo "Version updated. Next: jj commit -m 'Release' && jj git push"
