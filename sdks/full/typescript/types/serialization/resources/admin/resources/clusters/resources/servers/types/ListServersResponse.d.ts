/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../..";
import * as Rivet from "../../../../../../../../api";
import * as core from "../../../../../../../../core";
export declare const ListServersResponse: core.serialization.ObjectSchema<serializers.admin.clusters.ListServersResponse.Raw, Rivet.admin.clusters.ListServersResponse>;
export declare namespace ListServersResponse {
    interface Raw {
        servers: serializers.admin.clusters.Server.Raw[];
    }
}
