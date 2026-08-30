using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityBuildListUpdate
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntityBuildListUpdate); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntityBuildListUpdate)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize array BuildListModificationsIndices
            Rts.Serialization.Reference.Write(s, value.BuildListModificationsIndices, () =>
            {
                s.WriteVarInt32(value.BuildListModificationsIndices.Length);
                for(int i = 0 ; i < value.BuildListModificationsIndices.Length ; ++i)
                {
                    s.Write(value.BuildListModificationsIndices[i]);
                }
            });
            //  Serialize UnlockInBuildList
            s.Write(value.UnlockInBuildList);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EntityBuildListUpdate)) as Rts.CnC.Messages.Client.EntityBuildListUpdate;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize array BuildListModificationsIndices
            Rts.Serialization.Reference.Read(s, out value.BuildListModificationsIndices, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize UnlockInBuildList
            s.Read(out value.UnlockInBuildList);

            return value;
        }
        
    }
}
