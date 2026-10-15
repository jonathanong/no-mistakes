"""Validate a small raw capture; replay needs explicit external attestations."""
import argparse
import json
import resource

from decode import decode
from replay import replay


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('capture')
    parser.add_argument('--producer-events', type=int)
    parser.add_argument('--pid', type=int)
    parser.add_argument('--begin-index', type=int)
    parser.add_argument('--end-index', type=int)
    parser.add_argument('--continuous-capture', action='store_true')
    parser.add_argument('--object-history-verified', action='store_true')
    args = parser.parse_args()
    coordinates = (args.pid, args.begin_index, args.end_index)
    if any(value is not None for value in coordinates) and not all(value is not None for value in coordinates):
        parser.error('replay requires --pid, --begin-index and --end-index together')
    if args.producer_events is not None and not 0 < args.producer_events <= 100_000:
        parser.error('producer count must be between 1 and 100000')
    # Limits cover Python object expansion too, not only the zstd child.
    resource.setrlimit(resource.RLIMIT_AS, (256 * 1024**2, 256 * 1024**2))
    resource.setrlimit(resource.RLIMIT_CPU, (10, 10))
    events, report = decode(args.capture, expected_events=args.producer_events)
    if args.pid is not None:
        report['replay'] = replay(
            events, *coordinates, continuous_capture=args.continuous_capture,
            producer_verified=report['producer_count_verified'],
            history_verified=args.object_history_verified,
        )
    print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
