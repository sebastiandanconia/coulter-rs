#!/usr/bin/python3

import argparse
import requests
from google.protobuf import text_format
from btc_constants_pb2 import BtcConstantRequest, BtcConstantResponse

def parse_args():
    parser = argparse.ArgumentParser(description="Test client for BtcConstantService (Protobuf over HTTP)")
    parser.add_argument("--transaction_id", required=True, help="Transaction ID")
    parser.add_argument("--client_id", default="test_client", help="Client ID")
    parser.add_argument("--host", default="localhost", help="Server hostname")
    parser.add_argument("--port", default=3000, type=int, help="Server port")
    return parser.parse_args()

def main():
    args = parse_args()
    # Create the Protobuf request message
    request = BtcConstantRequest(
        transaction_id=args.transaction_id,
        client_id=args.client_id
    )
    # Serialize the request to bytes
    request_bytes = request.SerializeToString()
    # Construct the URL
    url = f"http://{args.host}:{args.port}/api/btc-constants/v1"
    headers = {"Content-Type": "application/x-protobuf"}
    try:
        # Send the POST request
        response = requests.post(url, data=request_bytes, headers=headers)
        response.raise_for_status()  # Raises an exception for non-200 status codes
        # Verify the response Content-Type
        if response.headers.get("Content-Type") != "application/x-protobuf":
            print("Error: Unexpected Content-Type:", response.headers.get("Content-Type"))
            return
        # Parse the response as a Protobuf message
        response_message = BtcConstantResponse()
        try:
            response_message.ParseFromString(response.content)
        except Exception as e:
            print("Error: Failed to parse response as Protobuf:", e)
            return
        # Convert to multi-line Protobuf text format and print
        response_str = text_format.MessageToString(response_message, as_one_line=False)
        print("Received response:")
        print(response_str)
    except requests.exceptions.RequestException as e:
        print("HTTP error:", e)

if __name__ == "__main__":
    main()