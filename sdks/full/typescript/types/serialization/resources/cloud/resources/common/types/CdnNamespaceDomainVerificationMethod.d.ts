/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";
export declare const CdnNamespaceDomainVerificationMethod: core.serialization.ObjectSchema<serializers.cloud.CdnNamespaceDomainVerificationMethod.Raw, Rivet.cloud.CdnNamespaceDomainVerificationMethod>;
export declare namespace CdnNamespaceDomainVerificationMethod {
    interface Raw {
        invalid?: serializers.EmptyObject.Raw | null;
        http?: serializers.cloud.CdnNamespaceDomainVerificationMethodHttp.Raw | null;
    }
}
