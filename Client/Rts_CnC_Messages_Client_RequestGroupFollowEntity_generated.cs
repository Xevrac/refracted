using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestGroupFollowEntity
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestGroupFollowEntity); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestGroupFollowEntity)obj;
            //  Serialize SrcPlayerId
            s.Write(value.SrcPlayerId);
            //  Serialize array SrcEntityIds
            Rts.Serialization.Reference.Write(s, value.SrcEntityIds, () =>
            {
                s.WriteVarInt32(value.SrcEntityIds.Length);
                for(int i = 0 ; i < value.SrcEntityIds.Length ; ++i)
                {
                    s.Write(value.SrcEntityIds[i]);
                }
            });
            //  Serialize TarPlayerId
            s.Write(value.TarPlayerId);
            //  Serialize TarEntityId
            s.Write(value.TarEntityId);
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestGroupFollowEntity)) as Rts.CnC.Messages.Client.RequestGroupFollowEntity;
            //  Deserialize SrcPlayerId
            s.Read(out value.SrcPlayerId);
            //  Deserialize array SrcEntityIds
            Rts.Serialization.Reference.Read(s, out value.SrcEntityIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize TarPlayerId
            s.Read(out value.TarPlayerId);
            //  Deserialize TarEntityId
            s.Read(out value.TarEntityId);
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
