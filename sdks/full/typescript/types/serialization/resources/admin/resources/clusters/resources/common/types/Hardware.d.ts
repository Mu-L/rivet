/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../..";
import * as Rivet from "../../../../../../../../api";
import * as core from "../../../../../../../../core";
export declare const Hardware: core.serialization.ObjectSchema<serializers.admin.clusters.Hardware.Raw, Rivet.admin.clusters.Hardware>;
export declare namespace Hardware {
    interface Raw {
        provider_hardware: string;
    }
}
