// This file was auto-generated by Fern from our API Definition.

package api

import (
	json "encoding/json"
	fmt "fmt"
	core "sdk/core"
	time "time"
)

type ErrorBody struct {
	Code          string         `json:"code"`
	Message       string         `json:"message"`
	RayId         string         `json:"ray_id"`
	Documentation *string        `json:"documentation,omitempty"`
	Metadata      *ErrorMetadata `json:"metadata,omitempty"`

	_rawJSON json.RawMessage
}

func (e *ErrorBody) UnmarshalJSON(data []byte) error {
	type unmarshaler ErrorBody
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*e = ErrorBody(value)
	e._rawJSON = json.RawMessage(data)
	return nil
}

func (e *ErrorBody) String() string {
	if len(e._rawJSON) > 0 {
		if value, err := core.StringifyJSON(e._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(e); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", e)
}

// Unstructured metadata relating to an error. Must be manually parsed.
type ErrorMetadata = interface{}

// RFC3339 timestamp
type Timestamp = time.Time

// Provided by watchable endpoints used in blocking loops.
type WatchResponse struct {
	// Index indicating the version of the data responded.
	// Pass this to `WatchQuery` to block and wait for the next response.
	Index string `json:"index"`

	_rawJSON json.RawMessage
}

func (w *WatchResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler WatchResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*w = WatchResponse(value)
	w._rawJSON = json.RawMessage(data)
	return nil
}

func (w *WatchResponse) String() string {
	if len(w._rawJSON) > 0 {
		if value, err := core.StringifyJSON(w._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(w); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", w)
}