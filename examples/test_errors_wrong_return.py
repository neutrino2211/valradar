# Test plugin: Wrong return type from init
# This should produce a clear error message about expecting a list

def init(args):
    # Wrong! Should return a list, but returning a string
    return "this is not a list"

def collect_data(ctx):
    return [ctx]

def process_data(ctx):
    return {"result": str(ctx)}

VALRADAR_CONFIG = {
    "metadata": {
        "name": "test-wrong-return",
        "version": "1.0.0",
        "description": "Test plugin with wrong return type"
    },
    "init": init,
    "collect_data": collect_data,
    "process_data": process_data
}
