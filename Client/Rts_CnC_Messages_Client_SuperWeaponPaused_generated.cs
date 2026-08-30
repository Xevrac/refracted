using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_SuperWeaponPaused
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.SuperWeaponPaused); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.SuperWeaponPaused)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize RemainingCooldownTimeMs
            s.Write(value.RemainingCooldownTimeMs);
            //  Serialize Paused
            s.Write(value.Paused);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.SuperWeaponPaused)) as Rts.CnC.Messages.Client.SuperWeaponPaused;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize RemainingCooldownTimeMs
            s.Read(out value.RemainingCooldownTimeMs);
            //  Deserialize Paused
            s.Read(out value.Paused);

            return value;
        }
        
    }
}
