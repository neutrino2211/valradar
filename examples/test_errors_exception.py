# Test plugin: Exception thrown in collect_data
# This should produce a clear error message with Python traceback

def init(args):
    return ["test_item"]

def helper_function():
    # This will raise an exception
    x = {}
    return x["nonexistent_key"]

def collect_data(ctx):
    # Call a helper that raises an exception
    result = helper_function()
    return [result]

def process_data(ctx):
    return {"result": str(ctx)}

VALRADAR_CONFIG = {
    "metadata": {
        "name": "test-exception",
        "version": "1.0.0",
        "description": "Test plugin that throws exception"
    },
    "init": init,
    "collect_data": collect_data,
    "process_data": process_data
}
