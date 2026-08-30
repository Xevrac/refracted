using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityExposed_AttackInfo
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntityExposed.AttackInfo); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntityExposed.AttackInfo)obj;
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Damage
            s.Write(value.Damage);
            //  Serialize WeaponSpecType
            s.Write(value.WeaponSpecType);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            Rts.CnC.Messages.Client.EntityExposed.AttackInfo value = default(Rts.CnC.Messages.Client.EntityExposed.AttackInfo);
            DeserializeValue(s, ref value);
            return value;
        }
        
        public static void DeserializeValue(System.IO.Stream s, ref Rts.CnC.Messages.Client.EntityExposed.AttackInfo value)
        {
            var valueRef = __makeref(value);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Damage
            s.Read(out value.Damage);
            //  Deserialize WeaponSpecType
            s.Read(out value.WeaponSpecType);

        }
    }
}
