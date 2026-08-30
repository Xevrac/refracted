using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_SuperWeaponExposed
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.SuperWeaponExposed); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.SuperWeaponExposed)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize AbilityCooldownTimeMs
            s.Write(value.AbilityCooldownTimeMs);
            //  Serialize RemainingCooldownTimeMs
            s.Write(value.RemainingCooldownTimeMs);
            //  Serialize PendingActivation
            s.Write(value.PendingActivation);
            //  Serialize RemainingActivationTimeMs
            s.Write(value.RemainingActivationTimeMs);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.SuperWeaponExposed)) as Rts.CnC.Messages.Client.SuperWeaponExposed;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize AbilityCooldownTimeMs
            s.Read(out value.AbilityCooldownTimeMs);
            //  Deserialize RemainingCooldownTimeMs
            s.Read(out value.RemainingCooldownTimeMs);
            //  Deserialize PendingActivation
            s.Read(out value.PendingActivation);
            //  Deserialize RemainingActivationTimeMs
            s.Read(out value.RemainingActivationTimeMs);

            return value;
        }
        
    }
}
