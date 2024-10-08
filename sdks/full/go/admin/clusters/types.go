// This file was auto-generated by Fern from our API Definition.

package clusters

import (
	json "encoding/json"
	fmt "fmt"
	uuid "github.com/google/uuid"
	core "sdk/core"
)

type BuildDeliveryMethod string

const (
	BuildDeliveryMethodTrafficServer BuildDeliveryMethod = "traffic_server"
	BuildDeliveryMethodS3Direct      BuildDeliveryMethod = "s3_direct"
)

func NewBuildDeliveryMethodFromString(s string) (BuildDeliveryMethod, error) {
	switch s {
	case "traffic_server":
		return BuildDeliveryMethodTrafficServer, nil
	case "s3_direct":
		return BuildDeliveryMethodS3Direct, nil
	}
	var t BuildDeliveryMethod
	return "", fmt.Errorf("%s is not a valid %T", s, t)
}

func (b BuildDeliveryMethod) Ptr() *BuildDeliveryMethod {
	return &b
}

type Cluster struct {
	ClusterId   uuid.UUID  `json:"cluster_id"`
	NameId      string     `json:"name_id"`
	CreateTs    int64      `json:"create_ts"`
	OwnerTeamId *uuid.UUID `json:"owner_team_id,omitempty"`

	_rawJSON json.RawMessage
}

func (c *Cluster) UnmarshalJSON(data []byte) error {
	type unmarshaler Cluster
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*c = Cluster(value)
	c._rawJSON = json.RawMessage(data)
	return nil
}

func (c *Cluster) String() string {
	if len(c._rawJSON) > 0 {
		if value, err := core.StringifyJSON(c._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(c); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", c)
}

type Datacenter struct {
	DatacenterId         uuid.UUID           `json:"datacenter_id"`
	ClusterId            uuid.UUID           `json:"cluster_id"`
	NameId               string              `json:"name_id"`
	DisplayName          string              `json:"display_name"`
	Provider             Provider            `json:"provider,omitempty"`
	ProviderDatacenterId string              `json:"provider_datacenter_id"`
	Pools                []*Pool             `json:"pools,omitempty"`
	BuildDeliveryMethod  BuildDeliveryMethod `json:"build_delivery_method,omitempty"`
	PrebakesEnabled      bool                `json:"prebakes_enabled"`

	_rawJSON json.RawMessage
}

func (d *Datacenter) UnmarshalJSON(data []byte) error {
	type unmarshaler Datacenter
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*d = Datacenter(value)
	d._rawJSON = json.RawMessage(data)
	return nil
}

func (d *Datacenter) String() string {
	if len(d._rawJSON) > 0 {
		if value, err := core.StringifyJSON(d._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(d); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", d)
}

type Hardware struct {
	ProviderHardware string `json:"provider_hardware"`

	_rawJSON json.RawMessage
}

func (h *Hardware) UnmarshalJSON(data []byte) error {
	type unmarshaler Hardware
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*h = Hardware(value)
	h._rawJSON = json.RawMessage(data)
	return nil
}

func (h *Hardware) String() string {
	if len(h._rawJSON) > 0 {
		if value, err := core.StringifyJSON(h._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(h); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", h)
}

type Pool struct {
	PoolType       PoolType    `json:"pool_type,omitempty"`
	Hardware       []*Hardware `json:"hardware,omitempty"`
	DesiredCount   int         `json:"desired_count"`
	MinCount       int         `json:"min_count"`
	MaxCount       int         `json:"max_count"`
	DrainTimeoutMs int64       `json:"drain_timeout_ms"`

	_rawJSON json.RawMessage
}

func (p *Pool) UnmarshalJSON(data []byte) error {
	type unmarshaler Pool
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*p = Pool(value)
	p._rawJSON = json.RawMessage(data)
	return nil
}

func (p *Pool) String() string {
	if len(p._rawJSON) > 0 {
		if value, err := core.StringifyJSON(p._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(p); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", p)
}

type PoolUpdate struct {
	PoolType     PoolType    `json:"pool_type,omitempty"`
	Hardware     []*Hardware `json:"hardware,omitempty"`
	DesiredCount *int        `json:"desired_count,omitempty"`
	MinCount     *int        `json:"min_count,omitempty"`
	MaxCount     *int        `json:"max_count,omitempty"`
	DrainTimeout *int64      `json:"drain_timeout,omitempty"`

	_rawJSON json.RawMessage
}

func (p *PoolUpdate) UnmarshalJSON(data []byte) error {
	type unmarshaler PoolUpdate
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*p = PoolUpdate(value)
	p._rawJSON = json.RawMessage(data)
	return nil
}

func (p *PoolUpdate) String() string {
	if len(p._rawJSON) > 0 {
		if value, err := core.StringifyJSON(p._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(p); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", p)
}

type Provider string

const (
	ProviderLinode Provider = "linode"
)

func NewProviderFromString(s string) (Provider, error) {
	switch s {
	case "linode":
		return ProviderLinode, nil
	}
	var t Provider
	return "", fmt.Errorf("%s is not a valid %T", s, t)
}

func (p Provider) Ptr() *Provider {
	return &p
}

type Server struct {
	ServerId     uuid.UUID `json:"server_id"`
	DatacenterId uuid.UUID `json:"datacenter_id"`
	PoolType     PoolType  `json:"pool_type,omitempty"`
	PublicIp     *string   `json:"public_ip,omitempty"`

	_rawJSON json.RawMessage
}

func (s *Server) UnmarshalJSON(data []byte) error {
	type unmarshaler Server
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*s = Server(value)
	s._rawJSON = json.RawMessage(data)
	return nil
}

func (s *Server) String() string {
	if len(s._rawJSON) > 0 {
		if value, err := core.StringifyJSON(s._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(s); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", s)
}
