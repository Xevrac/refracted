using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AoEEntityAttacked
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AoEEntityAttacked); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AoEEntityAttacked)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize TargetPlayerId
            s.Write(value.TargetPlayerId);
            //  Serialize TargetEntityId
            s.Write(value.TargetEntityId);
            //  Serialize Damage
            s.Write(value.Damage);
            //  Serialize DamageType
            s.Write(value.DamageType);
            //  Serialize WeaponSpecType
            s.Write(value.WeaponSpecType);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AoEEntityAttacked)) as Rts.CnC.Messages.Client.AoEEntityAttacked;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize TargetPlayerId
            s.Read(out value.TargetPlayerId);
            //  Deserialize TargetEntityId
            s.Read(out value.TargetEntityId);
            //  Deserialize Damage
            s.Read(out value.Damage);
            //  Deserialize DamageType
            s.Read(out value.DamageType);
            //  Deserialize WeaponSpecType
            s.Read(out value.WeaponSpecType);

            return value;
        }
        
    }
}
