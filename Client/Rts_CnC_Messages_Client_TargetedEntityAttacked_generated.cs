using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TargetedEntityAttacked
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TargetedEntityAttacked); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TargetedEntityAttacked)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize TargetPlayerId
            s.Write(value.TargetPlayerId);
            //  Serialize TargetEntityId
            s.Write(value.TargetEntityId);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize PotentialDamage
            s.Write(value.PotentialDamage);
            //  Serialize WeaponSpecType
            s.Write(value.WeaponSpecType);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TargetedEntityAttacked)) as Rts.CnC.Messages.Client.TargetedEntityAttacked;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize TargetPlayerId
            s.Read(out value.TargetPlayerId);
            //  Deserialize TargetEntityId
            s.Read(out value.TargetEntityId);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize PotentialDamage
            s.Read(out value.PotentialDamage);
            //  Deserialize WeaponSpecType
            s.Read(out value.WeaponSpecType);

            return value;
        }
        
    }
}
