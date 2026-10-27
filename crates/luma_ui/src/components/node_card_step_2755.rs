python3 -c "
import json
with open('/home/abdellah/.gemini/antigravity-cli/brain/6df659ea-a64c-4d8b-a7cc-ea6aed82f9ea/.system_generated/logs/transcript_full.jsonl') as f:
    for i, line in enumerate(f):
        if 'pub fn node_card_scene(' in line and 'ReplacementContent' in line:
            data = json.loads(line)
            for tc in data.get('tool_calls', []):
                args = tc.get('args', {})
                if 'ReplacementContent' in args and 'pub fn node_card_scene' in args['ReplacementContent']:
                    print(f'Found at step {i}')
                    with open('/home/abdellah/Univis-project/luma_editor/crates/luma_ui/src/components/node_card_original.rs', 'w') as out:
                        out.write(args['ReplacementContent'])
                    print('Wrote to node_card_original.rs')
                    break
"