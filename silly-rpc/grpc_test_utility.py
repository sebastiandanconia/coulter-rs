#!/usr/bin/python3

import argparse
import grpc
from google.protobuf import text_format
from btc_constants_pb2 import BtcConstantRequest
from btc_constants_pb2_grpc import BtcConstantServiceStub

def parse_args():
    parser = argparse.ArgumentParser(description="Test client for BtcConstantService")
    parser.add_argument("--transaction_id", required=True, help="Transaction ID")
    parser.add_argument("--client_id", default="test_client", help="Client ID")
    parser.add_argument("--host", default="localhost", help="Server hostname")
    parser.add_argument("--port", default=50051, type=int, help="Server port")
    return parser.parse_args()

def main():
    args = parse_args()
    request = BtcConstantRequest(
        transaction_id=args.transaction_id,
        client_id=args.client_id
    )
    with grpc.insecure_channel(f"{args.host}:{args.port}") as channel:
        stub = BtcConstantServiceStub(channel)
        try:
            response = stub.GetBtcConstants(request)
            response_str = text_format.MessageToString(response, as_one_line=False)
            print("Received response:")
            print(response_str)
        except grpc.RpcError as e:
            print(f"Error: {e}")

if __name__ == "__main__":
    main()