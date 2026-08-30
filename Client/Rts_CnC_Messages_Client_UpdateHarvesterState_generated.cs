using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_UpdateHarvesterState
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.UpdateHarvesterState); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.UpdateHarvesterState)obj;
            //  Serialize HarvesterPlayerId
            s.Write(value.HarvesterPlayerId);
            //  Serialize HarvesterId
            s.Write(value.HarvesterId);
            //  Serialize IsHarvesting
            s.Write(value.IsHarvesting);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.UpdateHarvesterState)) as Rts.CnC.Messages.Client.UpdateHarvesterState;
            //  Deserialize HarvesterPlayerId
            s.Read(out value.HarvesterPlayerId);
            //  Deserialize HarvesterId
            s.Read(out value.HarvesterId);
            //  Deserialize IsHarvesting
            s.Read(out value.IsHarvesting);

            return value;
        }
        
    }
}
