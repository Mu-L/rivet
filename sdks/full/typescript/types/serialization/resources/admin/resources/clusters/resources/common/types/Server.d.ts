/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../..";
import * as Rivet from "../../../../../../../../api";
import * as core from "../../../../../../../../core";
export declare const Server: core.serialization.ObjectSchema<serializers.admin.clusters.Server.Raw, Rivet.admin.clusters.Server>;
export declare namespace Server {
    interface Raw {
        server_id: string;
        public_ip: string;
    }
}
