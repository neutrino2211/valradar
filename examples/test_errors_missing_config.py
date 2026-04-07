# Test plugin: Missing VALRADAR_CONFIG
# This should produce a clear error message about the missing config

def init(args):
    return ["test"]

def collect_data(ctx):
    return [ctx]

def process_data(ctx):
    return {"result": str(ctx)}

# Intentionally missing VALRADAR_CONFIG!
