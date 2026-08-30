using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityResearchListUpdate
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntityResearchListUpdate); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntityResearchListUpdate)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize array ResearchListModificationsIndices
            Rts.Serialization.Reference.Write(s, value.ResearchListModificationsIndices, () =>
            {
                s.WriteVarInt32(value.ResearchListModificationsIndices.Length);
                for(int i = 0 ; i < value.ResearchListModificationsIndices.Length ; ++i)
                {
                    s.Write(value.ResearchListModificationsIndices[i]);
                }
            });
            //  Serialize UnlockInResearchList
            s.Write(value.UnlockInResearchList);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EntityResearchListUpdate)) as Rts.CnC.Messages.Client.EntityResearchListUpdate;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize array ResearchListModificationsIndices
            Rts.Serialization.Reference.Read(s, out value.ResearchListModificationsIndices, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize UnlockInResearchList
            s.Read(out value.UnlockInResearchList);

            return value;
        }
        
    }
}
