using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityExposed
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntityExposed); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntityExposed)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize EntityUnitId
            s.Write(value.EntityUnitId);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Orientation
            s.Write(value.Orientation);
            //  Serialize CurrentHealth
            s.Write(value.CurrentHealth);
            //  Serialize CurrentMaxHealth
            s.Write(value.CurrentMaxHealth);
            //  Serialize Placed
            s.Write(value.Placed);
            //  Serialize CreationFlags
            s.Write(value.CreationFlags);
            //  Serialize VisibilityState
            s.WriteEnum(value.VisibilityState);
            //  Serialize array Attacks
            Rts.Serialization.Reference.Write(s, value.Attacks, () =>
            {
                s.WriteVarInt32(value.Attacks.Length);
                for(int i = 0 ; i < value.Attacks.Length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityExposed_AttackInfo.Serializer.Serialize(s, value.Attacks[i]);
                }
            });
            //  Serialize IsNewEntity
            s.Write(value.IsNewEntity);
            //  Serialize array InstanceId
            Rts.Serialization.Reference.Write(s, value.InstanceId, () =>
            {
                s.WriteVarInt32(value.InstanceId.Length);
                s.Write(value.InstanceId, 0, value.InstanceId.Length);
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EntityExposed)) as Rts.CnC.Messages.Client.EntityExposed;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize EntityUnitId
            s.Read(out value.EntityUnitId);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Orientation
            s.Read(out value.Orientation);
            //  Deserialize CurrentHealth
            s.Read(out value.CurrentHealth);
            //  Deserialize CurrentMaxHealth
            s.Read(out value.CurrentMaxHealth);
            //  Deserialize Placed
            s.Read(out value.Placed);
            //  Deserialize CreationFlags
            s.Read(out value.CreationFlags);
            //  Deserialize VisibilityState
            s.ReadEnum(out value.VisibilityState);
            //  Deserialize array Attacks
            Rts.Serialization.Reference.Read(s, out value.Attacks, () =>
            {
                int length = s.ReadVarInt32();
                Rts.CnC.Messages.Client.EntityExposed.AttackInfo[] tmp = new Rts.CnC.Messages.Client.EntityExposed.AttackInfo[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityExposed_AttackInfo.Serializer.DeserializeValue(s, ref tmp[i]);
                }
                return tmp;
            });
            //  Deserialize IsNewEntity
            s.Read(out value.IsNewEntity);
            //  Deserialize array InstanceId
            Rts.Serialization.Reference.Read(s, out value.InstanceId, () =>
            {
                int length = s.ReadVarInt32();
                System.Byte[] tmp = new System.Byte[length];
                s.Read(tmp, 0, length);
                return tmp;
            });

            return value;
        }
        
    }
}
