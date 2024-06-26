/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";

export const BannedIdentity: core.serialization.ObjectSchema<
    serializers.group.BannedIdentity.Raw,
    Rivet.group.BannedIdentity
> = core.serialization.object({
    identity: core.serialization.lazyObject(async () => (await import("../../../../..")).identity.Handle),
    banTs: core.serialization.property(
        "ban_ts",
        core.serialization.lazy(async () => (await import("../../../../..")).Timestamp)
    ),
});

export declare namespace BannedIdentity {
    interface Raw {
        identity: serializers.identity.Handle.Raw;
        ban_ts: serializers.Timestamp.Raw;
    }
}
