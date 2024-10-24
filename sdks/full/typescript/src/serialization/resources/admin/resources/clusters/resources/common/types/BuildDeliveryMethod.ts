/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";

export const BuildDeliveryMethod: core.serialization.Schema<
    serializers.admin.clusters.BuildDeliveryMethod.Raw,
    Rivet.admin.clusters.BuildDeliveryMethod
> = core.serialization.enum_(["traffic_server", "s3_direct"]);

export declare namespace BuildDeliveryMethod {
    type Raw = "traffic_server" | "s3_direct";
}
