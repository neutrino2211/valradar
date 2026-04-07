# Test plugin: Missing required function (collect_data)
# This should produce a clear error message about the missing function

def init(args):
    return ["test"]

def process_data(ctx):
    return {"result": str(ctx)}

# Missing collect_data function!

VALRADAR_CONFIG = {
    "metadata": {
        "name": "test-missing-func",
        "version": "1.0.0",
        "description": "Test plugin with missing function"
    },
    "init": init,
    # "collect_data": missing!
    "process_data": process_data
}
