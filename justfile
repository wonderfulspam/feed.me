set quiet

spacefeeder_config_path := "spacefeeder.toml"

add_feed slug url author tier="new": (_check_binary_exists "dasel")
  dasel put -f {{spacefeeder_config_path}} -r toml -t string -v '{{url}}' 'feeds.{{slug}}.url' && \
  dasel put -f {{spacefeeder_config_path}} -r toml -t string -v '{{author}}' 'feeds.{{slug}}.author' && \
  dasel put -f {{spacefeeder_config_path}} -r toml -t string -v '{{tier}}' 'feeds.{{slug}}.tier'

pull_feed_info: build_spacefeeder
  spacefeeder fetch

build_spacefeeder:
  echo "Building spacefeeder"
  cd spacefeeder && cargo install --quiet --path . --locked

serve:
  zola serve

[no-exit-message]
_check_binary_exists binary_name:
  command -v {{binary_name}} > /dev/null 2>&1 || { echo "{{binary_name}} not found, aborting"; exit 1; }
